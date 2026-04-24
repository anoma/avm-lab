//! Core node logic: TCP listener, peer connections, inbound dispatch.
//!
//! # Architecture
//!
//! Each node runs several concurrent tasks:
//!
//! - **listener**: accepts incoming TCP connections, spawns a reader per peer.
//! - **peer reader**: reads frames from one peer, forwards to the inbound channel.
//! - **peer writer**: drains a per-peer mpsc channel, writes frames to the socket.
//! - **outbound router**: receives `OutboundMsg`s, fans out to per-peer writers.
//! - **inbound dispatch**: handles `Call`, `CallResponse`, `CreateNotify`, etc.
//!
//! # Threading model for !Send types
//!
//! `BehaviorFn` is `!Send`, so `BehaviorRegistry` and the `ITree` continuations
//! it produces are also `!Send`. We cannot move them into `tokio::spawn_blocking`
//! (which requires `Send`).
//!
//! Solution: a **dedicated interpreter thread** (plain `std::thread::spawn`, no
//! `Send` bound) owns both `State` and `BehaviorRegistry`. The async world
//! communicates with it via a `std::sync::mpsc` channel carrying `InterpRequest`
//! structs. Each request carries a `tokio::sync::oneshot::Sender` to deliver the
//! result back to the async caller.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{atomic::AtomicU64, Arc, Mutex};

use avm_core::instruction::Instruction;
use avm_core::itree::ITree;
use avm_core::store::{ObjectBehavior, Store};
use avm_core::types::{FreshIdGen, MachineId, ObjectId, ObjectMeta, Val};
use avm_core::vm::{BehaviorRegistry, State};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::directory::LocationDirectory;
use crate::protocol::{read_frame, write_frame, NodeMessage};
use crate::remote_call::{complete_pending, new_pending_map, PendingMap};
use crate::sse::EventBroadcaster;
use crate::transport::{OutboundMsg, TcpTransport};
use crate::NodeError;

// ─── Interpreter worker thread ────────────────────────────────────────────────

/// A request sent to the interpreter worker thread.
struct InterpRequest {
    /// Which object to call (looked up in State).
    target: ObjectId,
    /// Input message.
    input: Val,
    /// Caller object ID (for `get_sender`).
    sender: ObjectId,
    /// Deliver the result here.
    reply: oneshot::Sender<Result<Val, String>>,
}

/// A request to run a program produced by a factory closure on the interpreter thread.
///
/// We cannot ship an `ITree` across threads (it's `!Send`), but we *can* ship a
/// `Send` factory that produces the `ITree` on the worker thread where it will run.
struct ProgramRequest {
    /// Factory that builds the `ITree` on the interpreter thread.
    factory: Box<dyn FnOnce() -> ITree<Instruction, Val> + Send>,
    reply: oneshot::Sender<Result<(Val, avm_core::trace::Trace), String>>,
}

/// Commands the async world can send to the interpreter worker.
enum WorkerCmd {
    Call(InterpRequest),
    RunProgram(ProgramRequest),
    RegisterObject {
        id: ObjectId,
        behavior_name: String,
        machine: MachineId,
    },
}

/// Handle to the interpreter worker thread.
#[derive(Clone)]
pub struct WorkerHandle {
    tx: std::sync::mpsc::SyncSender<WorkerCmd>,
}

impl WorkerHandle {
    /// Submit a remote call to the interpreter thread and await the result.
    pub async fn call(
        &self,
        target: ObjectId,
        input: Val,
        sender: ObjectId,
    ) -> Result<Val, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(WorkerCmd::Call(InterpRequest {
                target,
                input,
                sender,
                reply: reply_tx,
            }))
            .map_err(|_| "interpreter thread gone".to_string())?;
        reply_rx
            .await
            .map_err(|_| "reply channel dropped".to_string())?
    }

    /// Submit a program factory to the interpreter thread and await its result.
    ///
    /// The factory is called on the worker thread, so the resulting `ITree`
    /// never needs to cross thread boundaries.
    pub async fn run_program(
        &self,
        factory: impl FnOnce() -> ITree<Instruction, Val> + Send + 'static,
    ) -> Result<(Val, avm_core::trace::Trace), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(WorkerCmd::RunProgram(ProgramRequest {
                factory: Box::new(factory),
                reply: reply_tx,
            }))
            .map_err(|_| "interpreter thread gone".to_string())?;
        reply_rx
            .await
            .map_err(|_| "reply channel dropped".to_string())?
    }

    /// Tell the interpreter thread to register an object in the shared state.
    pub fn register_object(&self, id: ObjectId, behavior_name: String, machine: MachineId) {
        let _ = self.tx.send(WorkerCmd::RegisterObject {
            id,
            behavior_name,
            machine,
        });
    }
}

/// Spawn the interpreter worker thread. Returns a handle the async code uses.
///
/// The worker owns `State` and `BehaviorRegistry`; neither needs to be `Send`.
fn spawn_interpreter_thread(
    mut state: State,
    registry: BehaviorRegistry,
    transport_factory: impl Fn() -> TcpTransport + Send + 'static,
) -> WorkerHandle {
    let (tx, rx) = std::sync::mpsc::sync_channel::<WorkerCmd>(256);
    std::thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                WorkerCmd::Call(req) => {
                    let result = (|| -> Result<Val, String> {
                        let program = {
                            let behavior_name = state
                                .store
                                .objects
                                .get(&req.target)
                                .map(|b| b.name.clone())
                                .ok_or_else(|| format!("object not found: {}", req.target))?;
                            registry
                                .resolve(&behavior_name, req.input.clone())
                                .map_err(|e| e.to_string())?
                        };
                        state.self_id = req.target;
                        state.input = req.input;
                        state.sender = Some(req.sender);
                        let transport = transport_factory();
                        avm_core::interpreter::interpret(program, &mut state, &registry, &transport)
                            .map(|s| s.value)
                            .map_err(|e| e.to_string())
                    })();
                    let _ = req.reply.send(result);
                }
                WorkerCmd::RunProgram(req) => {
                    let program = (req.factory)();
                    let transport = transport_factory();
                    let result = avm_core::interpreter::interpret(
                        program, &mut state, &registry, &transport,
                    )
                    .map(|s| (s.value, s.trace))
                    .map_err(|e| e.to_string());
                    let _ = req.reply.send(result);
                }
                WorkerCmd::RegisterObject {
                    id,
                    behavior_name,
                    machine,
                } => {
                    state
                        .store
                        .objects
                        .insert(id, ObjectBehavior::named(&behavior_name));
                    state.store.metadata.insert(
                        id,
                        ObjectMeta {
                            object_id: id,
                            machine: machine.clone(),
                            creating_controller: None,
                            current_controller: None,
                        },
                    );
                    state.store.states.insert(id, vec![]);
                }
            }
        }
    });
    WorkerHandle { tx }
}

// ─── Node config & builder ────────────────────────────────────────────────────

/// Configuration for a single AVM node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Human-readable name; used as the `MachineId`.
    pub name: String,
    /// TCP port to listen on.
    pub port: u16,
    /// Peer addresses to connect to at startup (`"host:port"`).
    pub peers: Vec<String>,
    /// Prefix for ID generation (bits [63:48] of every `ObjectId`).
    pub node_prefix: u16,
    /// TCP port for the SSE HTTP server. `None` disables the server.
    pub sse_port: Option<u16>,
}

/// A node before it is started.
pub struct Node {
    config: NodeConfig,
    registry: BehaviorRegistry,
}

impl Node {
    /// Create a new node (does not start listening yet).
    pub fn new(config: NodeConfig, registry: BehaviorRegistry) -> Self {
        Self { config, registry }
    }

    /// Start the node: bind the listener, connect to peers, spawn all tasks.
    pub async fn start(self) -> Result<RunningNode, NodeError> {
        let machine_id = MachineId(self.config.name.clone());

        let mut state = State::new(machine_id.clone());
        state.fresh_ids = FreshIdGen::with_prefix(self.config.node_prefix);

        // Shared async channels.
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<OutboundMsg>();
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel::<NodeMessage>();

        let pending_map: Arc<PendingMap> = Arc::new(new_pending_map());
        let request_counter = Arc::new(AtomicU64::new(0));
        let directory = LocationDirectory::new();

        // Build the transport factory the interpreter thread will use.
        let pm = Arc::clone(&pending_map);
        let rc = Arc::clone(&request_counter);
        let dir = directory.clone();
        let otx = outbound_tx.clone();
        let local = machine_id.clone();
        let transport_factory = move || TcpTransport {
            local_machine: local.clone(),
            outbound_tx: otx.clone(),
            pending_map: Arc::clone(&pm),
            request_counter: Arc::clone(&rc),
            directory: dir.clone(),
        };

        let worker = spawn_interpreter_thread(state, self.registry, transport_factory);

        // Per-peer outbound writer channels.
        let peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Connect to configured peers.
        for peer_addr in &self.config.peers {
            let peer_addr = peer_addr.clone();
            let inbound_tx2 = inbound_tx.clone();
            let pw2 = Arc::clone(&peer_writers);
            tokio::spawn(async move {
                connect_with_retry(&peer_addr, inbound_tx2, pw2).await;
            });
        }

        // Spawn outbound router.
        let pw_router = Arc::clone(&peer_writers);
        tokio::spawn(run_outbound_router(outbound_rx, pw_router));

        // Spawn inbound dispatcher.
        let worker2 = worker.clone();
        let pending2 = Arc::clone(&pending_map);
        let dir2 = directory.clone();
        let otx2 = outbound_tx.clone();
        tokio::spawn(run_inbound_dispatch(
            inbound_rx, worker2, pending2, dir2, otx2,
        ));

        // Bind TCP listener.
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port)
            .parse()
            .expect("valid socket address");
        let listener = TcpListener::bind(addr).await?;
        info!(name = %self.config.name, port = self.config.port, "node listening");

        // Spawn listener task.
        let inbound_tx3 = inbound_tx.clone();
        let pw_listener = Arc::clone(&peer_writers);
        tokio::spawn(run_listener(listener, inbound_tx3, pw_listener));

        // Create the SSE event broadcaster (capacity: 256 queued messages).
        let broadcaster = Arc::new(EventBroadcaster::new(256));

        // Optionally spawn the SSE HTTP server.
        if let Some(sse_port) = self.config.sse_port {
            let router = crate::sse::sse_router_arc(Arc::clone(&broadcaster));
            let sse_addr: SocketAddr = format!("0.0.0.0:{sse_port}")
                .parse()
                .expect("valid SSE socket address");
            info!(port = sse_port, "SSE server listening");
            tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind(sse_addr)
                    .await
                    .expect("SSE listener bind failed");
                axum::serve(listener, router)
                    .await
                    .expect("SSE server error");
            });
        }

        Ok(RunningNode {
            name: self.config.name,
            port: self.config.port,
            peers: self.config.peers,
            worker,
            directory,
            pending_map,
            request_counter,
            outbound_tx,
            peer_writers,
            broadcaster,
        })
    }
}

// ─── RunningNode ─────────────────────────────────────────────────────────────

/// A node that has been started and has all tasks running.
pub struct RunningNode {
    pub name: String,
    #[allow(dead_code)]
    pub port: u16,
    #[allow(dead_code)]
    pub peers: Vec<String>,
    worker: WorkerHandle,
    pub directory: LocationDirectory,
    #[allow(dead_code)]
    pub pending_map: Arc<PendingMap>,
    #[allow(dead_code)]
    pub request_counter: Arc<AtomicU64>,
    pub outbound_tx: mpsc::UnboundedSender<OutboundMsg>,
    pub peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
    /// Broadcaster for real-time SSE trace events.
    pub broadcaster: Arc<EventBroadcaster>,
}

impl RunningNode {
    /// Pre-populate the store with an object so it can receive calls.
    pub fn register_object(&self, id: ObjectId, behavior_name: &str) {
        let machine = MachineId(self.name.clone());
        self.directory.insert(id, machine.clone());
        self.worker
            .register_object(id, behavior_name.to_string(), machine);
    }

    /// Announce that an object on this node exists to all peers.
    pub fn announce_object(&self, id: ObjectId) {
        let msg = NodeMessage::CreateNotify {
            object_id: id,
            machine_id: MachineId(self.name.clone()),
        };
        let _ = self.outbound_tx.send(OutboundMsg {
            target_machine: MachineId("*".into()),
            message: msg,
        });
    }

    /// Run an orchestrator program on the interpreter thread and return the result.
    ///
    /// Accepts a factory closure (which is `Send`) that produces the `ITree`
    /// on the interpreter thread, avoiding the `!Send` constraint on `ITree`.
    ///
    /// After the program completes, each [`avm_core::trace::LogEntry`] in the
    /// resulting trace is serialised to JSON and published to all active SSE
    /// subscribers via the [`EventBroadcaster`].
    pub async fn run_program(
        &self,
        factory: impl FnOnce() -> ITree<Instruction, Val> + Send + 'static,
    ) -> Result<avm_core::interpreter::Success<Val>, NodeError> {
        let (value, trace) = self
            .worker
            .run_program(factory)
            .await
            .map_err(NodeError::Interpret)?;

        // Broadcast each trace entry to SSE subscribers.
        for entry in &trace {
            if let Ok(json) = serde_json::to_string(entry) {
                self.broadcaster.publish(json);
            }
        }

        Ok(avm_core::interpreter::Success { value, trace })
    }

    /// Wait until at least one entry for each peer name appears in the
    /// peer-writers map (i.e., TCP connections are established).
    pub async fn wait_for_peers(&self, peer_names: &[&str]) {
        loop {
            {
                let pw = self
                    .peer_writers
                    .lock()
                    .expect("peer_writers lock poisoned");
                if peer_names.iter().all(|n| pw.contains_key(*n)) {
                    return;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait until a specific object appears in the location directory.
    pub async fn wait_for_object(&self, id: ObjectId) {
        loop {
            if self.directory.lookup(id).is_some() {
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}

// ─── Async tasks ─────────────────────────────────────────────────────────────

/// Accept incoming TCP connections forever, spawning reader/writer tasks.
async fn run_listener(
    listener: TcpListener,
    inbound_tx: mpsc::UnboundedSender<NodeMessage>,
    peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
) {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!(%addr, "accepted connection");
                let inbound_tx2 = inbound_tx.clone();
                let pw = Arc::clone(&peer_writers);
                tokio::spawn(handle_accepted(stream, addr, inbound_tx2, pw));
            }
            Err(e) => {
                error!("accept error: {e}");
            }
        }
    }
}

/// Handle one accepted TCP connection: split into reader/writer halves.
async fn handle_accepted(
    stream: TcpStream,
    addr: SocketAddr,
    inbound_tx: mpsc::UnboundedSender<NodeMessage>,
    peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
) {
    let peer_key = addr.to_string();
    let (reader, writer) = stream.into_split();
    let (tx, rx) = mpsc::unbounded_channel::<NodeMessage>();

    {
        peer_writers
            .lock()
            .expect("peer_writers lock poisoned")
            .insert(peer_key.clone(), tx);
    }

    let pw2 = Arc::clone(&peer_writers);
    tokio::spawn(run_peer_writer(writer, rx, peer_key.clone(), pw2));
    run_peer_reader(reader, inbound_tx, peer_key).await;
}

/// Read frames from one peer and forward them to the inbound channel.
async fn run_peer_reader(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    inbound_tx: mpsc::UnboundedSender<NodeMessage>,
    peer_key: String,
) {
    loop {
        match read_frame(&mut reader).await {
            Ok(Some(msg)) => {
                if inbound_tx.send(msg).is_err() {
                    break;
                }
            }
            Ok(None) => {
                info!(peer = %peer_key, "connection closed");
                break;
            }
            Err(e) => {
                warn!(peer = %peer_key, "read error: {e}");
                break;
            }
        }
    }
}

/// Write frames to one peer, draining a per-peer mpsc channel.
async fn run_peer_writer(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::UnboundedReceiver<NodeMessage>,
    peer_key: String,
    peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
) {
    while let Some(msg) = rx.recv().await {
        if let Err(e) = write_frame(&mut writer, &msg).await {
            warn!(peer = %peer_key, "write error: {e}");
            break;
        }
    }
    let _ = writer.shutdown().await;
    peer_writers
        .lock()
        .expect("peer_writers lock poisoned")
        .remove(&peer_key);
    debug!(peer = %peer_key, "writer task exiting");
}

/// Route outbound messages to the correct per-peer writer channel.
///
/// A `target_machine` of `"*"` means broadcast to all current peers.
async fn run_outbound_router(
    mut rx: mpsc::UnboundedReceiver<OutboundMsg>,
    peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
) {
    while let Some(msg) = rx.recv().await {
        if msg.target_machine.0 == "*" {
            // Broadcast.
            let senders: Vec<_> = peer_writers
                .lock()
                .expect("peer_writers lock poisoned")
                .values()
                .cloned()
                .collect();
            for tx in senders {
                let _ = tx.send(msg.message.clone());
            }
        } else {
            let target = msg.target_machine.0.clone();
            let pw = peer_writers.lock().expect("peer_writers lock poisoned");
            if let Some(tx) = pw.get(&target) {
                let _ = tx.send(msg.message);
            } else {
                // Search for a key whose host part matches (handles "host:port" keys).
                let found = pw
                    .iter()
                    .find(|(k, _)| k.split(':').next() == Some(&target));
                if let Some((_, tx)) = found {
                    let _ = tx.send(msg.message);
                } else {
                    warn!(target = %target, "no peer writer found for target");
                }
            }
        }
    }
}

/// Dispatch inbound messages: run calls, complete responses, update directory.
async fn run_inbound_dispatch(
    mut rx: mpsc::UnboundedReceiver<NodeMessage>,
    worker: WorkerHandle,
    pending_map: Arc<PendingMap>,
    directory: LocationDirectory,
    outbound_tx: mpsc::UnboundedSender<OutboundMsg>,
) {
    while let Some(msg) = rx.recv().await {
        match msg {
            NodeMessage::Call {
                request_id,
                target,
                input,
                sender,
                sender_machine,
            } => {
                let worker2 = worker.clone();
                let pending2 = Arc::clone(&pending_map);
                let otx = outbound_tx.clone();
                tokio::spawn(async move {
                    let result = worker2.call(target, input, sender).await;
                    // Deliver response to the caller.
                    let response = NodeMessage::CallResponse { request_id, result };
                    let _ = otx.send(OutboundMsg {
                        target_machine: sender_machine,
                        message: response,
                    });
                    // Also complete any local pending entry (for symmetry; usually absent).
                    complete_pending(&pending2, request_id, Ok(Val::Nothing));
                });
            }

            NodeMessage::CallResponse { request_id, result } => {
                complete_pending(&pending_map, request_id, result);
            }

            NodeMessage::CreateNotify {
                object_id,
                machine_id,
            } => {
                info!(%object_id, %machine_id, "directory: object registered");
                directory.insert(object_id, machine_id);
            }

            NodeMessage::DestroyNotify { object_id } => {
                info!(%object_id, "directory: object removed");
                directory.remove(object_id);
            }
        }
    }
}

/// Attempt to connect to a peer, retrying every second until successful.
async fn connect_with_retry(
    peer_addr: &str,
    inbound_tx: mpsc::UnboundedSender<NodeMessage>,
    peer_writers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<NodeMessage>>>>,
) {
    loop {
        match TcpStream::connect(peer_addr).await {
            Ok(stream) => {
                info!(peer = %peer_addr, "connected to peer");
                // Key by the hostname portion so the router matches on machine name.
                let peer_key = peer_addr.split(':').next().unwrap_or(peer_addr).to_string();
                let (reader, writer) = stream.into_split();
                let (tx, rx) = mpsc::unbounded_channel::<NodeMessage>();
                {
                    peer_writers
                        .lock()
                        .expect("peer_writers lock poisoned")
                        .insert(peer_key.clone(), tx);
                }
                let pw2 = Arc::clone(&peer_writers);
                tokio::spawn(run_peer_writer(writer, rx, peer_key.clone(), pw2));
                run_peer_reader(reader, inbound_tx.clone(), peer_key).await;
                warn!(peer = %peer_addr, "peer disconnected, will retry");
            }
            Err(e) => {
                debug!(peer = %peer_addr, "connect failed ({e}), retrying in 1s");
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

// ─── Utility ─────────────────────────────────────────────────────────────────

/// Build a minimal state pre-seeded with the given store contents.
#[allow(dead_code)]
pub fn state_with_store(machine_id: MachineId, store: Store, prefix: u16) -> State {
    let mut state = State::new(machine_id);
    state.store = store;
    state.fresh_ids = FreshIdGen::with_prefix(prefix);
    state
}
