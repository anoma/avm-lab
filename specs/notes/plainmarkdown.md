# Notes on the current avm

The goal is to quickly answer for each command

1. relevance to object views
2. relevance to commiting events over object to the virtual state, comes in two flavours: shielded and (fully) transparent
3. clarity of why we need versions for 1. or 2.


## [createobj](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#createobj)

1. potentially we want to create objects view for playing around with them independent of any finalized object ids
2. creating a new object typically is fully autonomous and will create a commitement to the virtual state; concretely, we re-use the goose approach, relying on the nullifier mechanics
3. 2. is clear, 1. is a nice to have

If we speak about a runtime object, what is the qualification `runtime` needed for? It seems that the runtime has an off-controller part (in p2p land) and and an on contoller part.

## [destroyobj](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#destroyobj)

as for the previous [createobj](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#createobj)


## [call](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#call)

1. optimistic execution of a (fully specified) call---may use a temporary representation for a composed object (in the runtime)
2. check for possibility of a (fully specified) call---needs to be affecting the relevant persistent object in virtual stat; involves resource logic encoding and proofs, e.g., as in goose
3. no question that we need this one; relies on the isolated turn principle for a single object (which may be a composite of several ones)

## [receive](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#receive)

1. this may be necessary to figure out the precise trace/tx that is to be pushed to the controller
2. on controller, I am not sure why we would need this instruction; this should be compiled away once we hit the controller, this may be (ephemeral) messages (??? as in goose???)
3. in p2p land, it would probably be cleaner to represent the set of available messages as in the work of agha, but I guess, that gives already the reason why we would want sth. like this instruction, potentially syntactically sugared in an [agha-eseque](https://osl.cs.illinois.edu/media/papers/agha-1997-abstracting_interaction_patterns.pdf) language

## [self](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#self)

1. interestingly, self need not even return a UUID; it is just relevant that the object type is known and its history; this serves to send message to self
2. similarly, the actual id may not be necessary?
3. The main point seems to be able to allow different object views to communicate with each other? on the other hand, https://forum.anoma.net/t/resources-as-purely-functional-objects/1455 talks about resources on controllers and an abstraction of resources as objects

## [input](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#input)

1. makes a lot of sense if we are exploring a branch optimisticall
2. may be relevant for an interpretation based execution on the controller
3. this may be mostly relevant in the p2p land, should rather be in the runtime/execution context on the controller?


## [getcurrentmachine](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#getcurrentmachine)

1. clearly, is relevant for p2p purposes
2. irrelevant on the controller?
3. This instruction shows clearly that some of the commands do have very different purposes in different phases of the [AVM-cycle](https://excalidraw.com/#json=364-4qrOOKqjtROwL3iNm,z7eCRaY8bS53mOM4cPwJVw)

## [history](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#history)

1. for object views, this typically is condensed into the behavioural state / a concrete memory presentation of the object; this may be a history, but that's an AVM specific choice?
2. on the controller, this is almost superfluous? The object version is the same as the history (besides the OID)?
3. important to clarify the role of history, pruning of history for efficiency reasons

## [sender](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#sender)

1. this gives (indirect) information about which objects eventually need to sync in an event
2. once we are on the controller, we have a completed msc, and this is just a lookup in the msc?
3. on access control: so far, it seems we have no explicit notion of subject/principal? 

## [scrymeta-unsafe](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#scrymeta-unsafe)

1. reaching out for potentially relevan object views already circulating or still in the "freezer" 
2. access to the *current version* relative to (filter/object id)
3. the main question concerns how to make sense of several matches
   - first/best match only?
   - execute "for all"
   
on the higher levels, we only have filters, thus could hide this under syntactic sugar; could this made be safe, somewhat how graphQL queries are made safe if we query github?

## [scrydeep-unsafe](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#scrydeep-unsafe)

1. this seems to mainly be about p2p land
2. probably not on the controller ?
3. ??? in p2p land, this may just return arbitrary magic stuff ???

## [begintx](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#begintx)

1. "very speculative" in p2p land?
2. begin of the actual tx to be executed
3. only necessary on the controller and for pure speculation in p2p land?

## [committx](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#committx)

1. simulating a transaction before we send it ?
2. amounts to success of a tx?
3. fairly clearn on controller and makes sense for simulation w.r.t. to current local view

## [aborttx](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#aborttx)

1. failed simulation, no harm done
2. aka as revert in Ethereum
3. farily clear

--- 

## on the [tx layer/control](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#transaction-control)

this is in analgoy to textbook material on database theory?

---

## on [pure functions](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#pure-function-instructions)

in short, this is a nice way to implement object specifications 

--- 

## on [machine instructions](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#machine-instructions)

1. that's just p2p land

right?

## [controller-instructions](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#controller-instructions)

this occurs in the short term only as single tx or at the end of a transaction after effecting a change, 
i.e., 
all these are added as parameters to commitTx? 

- thaw could be a special case of scry in p2p land?

- freeze for a single object is just the a special case of sending a tx?

## [finite-domain-constraint-programming-instructions](http://127.0.0.1:8000/avm-lab/specs/AVM/Instruction.html#finite-domain-constraint-programming-instructions)

This is probably just a PoC for what we under the hood to to figure out good possibilities for which programs to execute on which objects. 
I would not make thes explicit instructions, but non-deterministic transitions that just happen "spontaneously" in p2p land / on the solver. 















