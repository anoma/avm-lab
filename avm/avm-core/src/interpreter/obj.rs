//! Object instruction handler: create, destroy, call, receive.

use crate::error::{AVMError, ObjError};
use crate::instruction::ObjInstruction;
use crate::interpreter::helpers::{log_event, with_call_context};
use crate::interpreter::interpret;
use crate::trace::{EventType, Trace};
use crate::types::{ObjectId, Val};
use crate::vm::state::CreateEntry;
use crate::vm::{BehaviorRegistry, State};

pub fn execute(
    instr: ObjInstruction,
    state: &mut State,
    registry: &BehaviorRegistry,
) -> Result<(Val, Trace), AVMError> {
    match instr {
        ObjInstruction::Create {
            behavior_name,
            controller,
        } => execute_create(behavior_name, controller, state),
        ObjInstruction::Destroy(id) => execute_destroy(id, state),
        ObjInstruction::Call { target, input } => execute_call(target, input, state, registry),
        ObjInstruction::Receive => execute_receive(state),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn execute_create(
    behavior_name: String,
    controller: Option<crate::types::ControllerId>,
    state: &mut State,
) -> Result<(Val, Trace), AVMError> {
    let new_id = state.fresh_ids.next_object_id();
    let mut trace = Vec::new();

    if state.in_transaction() {
        state.creates.insert(
            new_id,
            CreateEntry {
                id: new_id,
                behavior_name: behavior_name.clone(),
                controller,
            },
        );
    } else {
        // Outside tx: create immediately
        let meta = crate::types::ObjectMeta {
            object_id: new_id,
            machine: state.machine_id.clone(),
            creating_controller: controller.clone(),
            current_controller: controller,
        };
        state
            .store
            .objects
            .insert(new_id, crate::store::ObjectBehavior::named(&behavior_name));
        state.store.metadata.insert(new_id, meta);
        state.store.states.insert(new_id, vec![]);
    }

    trace.push(log_event(
        state,
        EventType::ObjectCreated {
            id: new_id,
            behavior_name,
        },
    ));

    // Return the new object ID as an ObjectRef Val
    Ok((Val::ObjectRef(new_id), trace))
}

fn execute_destroy(id: ObjectId, state: &mut State) -> Result<(Val, Trace), AVMError> {
    let exists = state.store.objects.contains(&id) || state.creates.contains_key(&id);

    if !exists {
        return Err(ObjError::NotFound(id).into());
    }

    let mut trace = Vec::new();
    if state.in_transaction() {
        state.destroys.insert(id);
    } else {
        state.store.remove(&id);
    }

    trace.push(log_event(state, EventType::ObjectDestroyed(id)));
    Ok((Val::Bool(true), trace))
}

fn execute_call(
    target: ObjectId,
    input: Val,
    state: &mut State,
    registry: &BehaviorRegistry,
) -> Result<(Val, Trace), AVMError> {
    // Look up the target's behavior
    let behavior_name = state
        .store
        .objects
        .get(&target)
        .map(|b| b.name.clone())
        .or_else(|| state.creates.get(&target).map(|c| c.behavior_name.clone()))
        .ok_or(ObjError::NotFound(target))?;

    let target_machine = state
        .store
        .metadata
        .get(&target)
        .map_or_else(|| state.machine_id.clone(), |m| m.machine.clone());

    // Record in tx log if in transaction
    if state.in_transaction() {
        state.tx_log.push((target, input.clone()));
        state.observe(target);
    }

    // Build the callee program via the registry
    let callee_program = registry.resolve(&behavior_name, input.clone())?;

    // Execute in the callee's context, mutating state in-place (no clone).
    let caller_id = state.self_id;
    let result = with_call_context(
        state,
        target,
        input.clone(),
        caller_id,
        target_machine,
        |callee_state| interpret(callee_program, callee_state, registry),
    );

    let mut trace = Vec::new();
    if let Ok(success) = result {
        trace.extend(success.trace);

        let output = success.value.clone();
        trace.push(log_event(
            state,
            EventType::ObjectCalled {
                id: target,
                input,
                output: Some(output.clone()),
            },
        ));
        Ok((Val::just(output), trace))
    } else {
        trace.push(log_event(
            state,
            EventType::ObjectCalled {
                id: target,
                input,
                output: None,
            },
        ));
        Ok((Val::Nothing, trace))
    }
}

#[allow(clippy::unnecessary_wraps)]
fn execute_receive(state: &mut State) -> Result<(Val, Trace), AVMError> {
    // In this single-threaded interpreter, receive returns Nothing.
    // A distributed runtime would block or dequeue from a mailbox.
    let trace = vec![log_event(
        state,
        EventType::MessageReceived {
            id: state.self_id,
            input: Val::Nothing,
        },
    )];
    Ok((Val::Nothing, trace))
}
